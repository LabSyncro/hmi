import { invoke } from '@tauri-apps/api/core';

export interface JoinParams<T = unknown> {
  table: string
  left_column: keyof T extends string ? keyof T : string
  right_column: string
  kind: 'inner' | 'left' | 'right'
  alias?: string
  parent_table?: string
}

export interface QueryParams<T> {
  table: string
  columns?: string[]
  conditions?: [string, unknown][]
  order_by?: [string, boolean][]
  limit?: number
  offset?: number
  joins?: JoinParams<T>[]
}

export interface RawQueryParams {
  sql: string
  params?: unknown[]
}

export interface InsertParams<T> {
  table: string
  value: Partial<T>
}

export interface DbClient {
  query<T>(params: QueryParams<T>): Promise<T[]>
  queryRaw<T>(params: RawQueryParams): Promise<T[]>
}

class TauriDbClient implements DbClient {
  async query<T>(params: QueryParams<T>): Promise<T[]> {
    try {
      return await invoke<T[]>('query_table', {
        params: {
          table: params.table,
          columns: params.columns,
          conditions: params.conditions,
          order_by: params.order_by,
          limit: params.limit,
          offset: params.offset,
          joins: params.joins
        },
      });
    } catch (error) {
      throw error;
    }
  }

  async queryRaw<T>(params: RawQueryParams): Promise<T[]> {
    try {
      return await invoke<T[]>('query_raw', {
        params: {
          sql: params.sql,
          params: params.params || []
        },
      });
    } catch (error) {
      throw error;
    }
  }
}

export const db = new TauriDbClient();

export class DatabaseClient {
  private static instance: DatabaseClient;
  private constructor() { }

  public static getInstance(): DatabaseClient {
    if (!DatabaseClient.instance) {
      DatabaseClient.instance = new DatabaseClient();
    }
    return DatabaseClient.instance;
  }

  public async syncSchema(): Promise<void> {
    await invoke('sync_schema');
  }

  public async insert<T>(params: InsertParams<T>): Promise<T> {
    return invoke<T>('insert_into_table', {
      params: {
        table: params.table,
        value: params.value,
      },
    });
  }

  public table<T>(name: string): TableQueryBuilder<T> {
    return new TableQueryBuilder<T>(this, name);
  }

  public async query<T>(params: QueryParams<T>): Promise<T[]> {
    return db.query(params);
  }

  public async queryRaw<T>(params: RawQueryParams): Promise<T[]> {
    return db.queryRaw(params);
  }
}

export class TableQueryBuilder<T> {
  private columns?: (keyof T)[];
  private conditions: Array<[keyof T, unknown]> = [];
  private orderByColumns: Array<[keyof T, boolean]> = [];
  private limitValue?: number;
  private offsetValue?: number;

  constructor(
    private readonly client: DatabaseClient,
    private readonly tableName: string,
  ) { }

  select(...columns: (keyof T)[]): this {
    this.columns = columns;
    return this;
  }

  where(column: keyof T, value: unknown): this {
    this.conditions.push([column, value]);
    return this;
  }

  orderBy(column: keyof T, ascending = true): this {
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
    return this.client.query<T>({
      table: this.tableName,
      columns: this.columns?.map(String),
      conditions: this.conditions.map(([col, val]) => [String(col), val]),
      order_by: this.orderByColumns.map(([col, asc]) => [String(col), asc]),
      limit: this.limitValue,
      offset: this.offsetValue,
    });
  }

  async insert(value: Partial<T>): Promise<T> {
    return this.client.insert<T>({
      table: this.tableName,
      value,
    });
  }
} 