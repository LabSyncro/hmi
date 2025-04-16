import { invoke } from "@tauri-apps/api/core";

export type JoinParams = {
  table: string;
  left_column: string;
  right_column: string;
  kind: "inner" | "left" | "right";
  alias?: string;
  parent_table?: string;
};

export type QueryParams = {
  table: string;
  columns?: string[];
  conditions?: [string, unknown][];
  order_by?: [string, boolean][];
  limit?: number;
  offset?: number;
  joins?: JoinParams[];
};

export type RawQueryParams = {
  sql: string;
  params?: unknown[];
};

export type InsertParams<T> = {
  table: string;
  value: Partial<T>;
};

export interface DbClient {
  queryRaw<T>(params: RawQueryParams): Promise<T[]>;
  table<T>(name: string): TableQueryBuilder<T>;
  insert<T>(params: InsertParams<T>): Promise<T>;
}

class TauriDbClient implements DbClient {
  table<T>(name: string): TableQueryBuilder<T> {
    return new TableQueryBuilder<T>(this, name);
  }

  async query<T>(params: QueryParams): Promise<T[]> {
    try {
      return await invoke<T[]>("query_table", {
        params: {
          table: params.table,
          columns: params.columns,
          conditions: params.conditions,
          order_by: params.order_by,
          limit: params.limit,
          offset: params.offset,
          joins: params.joins,
        },
      });
    } catch (error) {
      throw error;
    }
  }

  async queryRaw<T>(params: RawQueryParams): Promise<T[]> {
    try {
      return await invoke<T[]>("query_raw", {
        params: {
          sql: params.sql,
          params: params.params || [],
        },
      });
    } catch (error) {
      throw error;
    }
  }

  async insert<T>(params: InsertParams<T>): Promise<T> {
    try {
      const tableName = params.table.includes(".")
        ? params.table
        : `public.${params.table}`;

      return await invoke<T>("insert_into_table", {
        params: {
          table: tableName,
          value: params.value,
        },
      });
    } catch (error) {
      throw error;
    }
  }
}

export const db = new TauriDbClient();

export async function syncSchema(): Promise<void> {
  await invoke("sync_schema");
}

export class TableQueryBuilder<T> {
  private columns?: string[];
  private conditions: Array<[string, unknown]> = [];
  private orderByColumns: Array<[string, boolean]> = [];
  private limitValue?: number;
  private offsetValue?: number;
  private includeRelations: Array<{
    table: string;
    as?: string;
    select?: string[];
    on: {
      from: string;
      to: string;
    };
  }> = [];

  constructor(
    private readonly client: TauriDbClient,
    private readonly tableName: string
  ) {}

  select(columns: string[]): this {
    this.columns = columns;
    return this;
  }

  include(params: {
    table: string;
    as?: string;
    select?: string[];
    on: {
      from: string;
      to: string;
    };
  }): this {
    this.includeRelations.push(params);
    return this;
  }

  where(column: string, value: unknown): this {
    this.conditions.push([column, value]);
    return this;
  }

  whereMany(conditions: Record<string, unknown>): this {
    Object.entries(conditions).forEach(([column, value]) => {
      this.conditions.push([column, value]);
    });
    return this;
  }

  orderBy(column: string, ascending = true): this {
    this.orderByColumns.push([column, ascending]);
    return this;
  }

  limit(limit: number): this {
    this.limitValue = limit;
    return this;
  }

  offset(offset: number): this {
    this.offsetValue = offset;
    return this;
  }

  async execute(): Promise<T[]> {
    const joins = this.includeRelations.map((relation) => ({
      table: relation.table,
      left_column: relation.on.from,
      right_column: relation.on.to,
      kind: "left" as const,
    }));

    const allColumns = [
      ...(this.columns || []),
      ...this.includeRelations.flatMap((relation) =>
        (relation.select || []).map(
          (col) => `${relation.table}.${col} AS ${relation.table}_${col}`
        )
      ),
    ];

    return this.client.query<T>({
      table: this.tableName,
      columns: allColumns.length > 0 ? allColumns : undefined,
      conditions: this.conditions,
      order_by:
        this.orderByColumns.length > 0 ? this.orderByColumns : undefined,
      limit: this.limitValue,
      offset: this.offsetValue,
      joins: joins.length > 0 ? joins : undefined,
    });
  }

  async first(): Promise<T | null> {
    this.limit(1);
    const results = await this.execute();
    return results[0] || null;
  }

  async insert(value: Partial<T>): Promise<T> {
    return this.client.insert<T>({
      table: this.tableName,
      value,
    });
  }
}
