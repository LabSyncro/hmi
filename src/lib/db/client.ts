import { invoke } from '@tauri-apps/api/core';

export interface QueryParams<T = any> {
  table: string;
  columns?: (keyof T)[];
  conditions?: Array<[keyof T, any]>;
  orderBy?: Array<[keyof T, boolean]>;
  limit?: number;
  offset?: number;
}

export interface InsertParams<T> {
  table: string;
  value: Partial<T>;
}

export class DatabaseClient {
  private static instance: DatabaseClient;

  private constructor() {
    console.log('DatabaseClient instance created');
    console.warn(
      "DEVELOPER NOTE: The Tauri backend needs to be updated to properly manage state.\n" +
      "In the Rust backend, ensure you call .manage() on the state before using database commands.\n" +
      "This is typically done in the main.rs file where the Tauri app is initialized."
    );
  }

  public static getInstance(): DatabaseClient {
    if (!DatabaseClient.instance) {
      console.log('Creating new DatabaseClient instance');
      DatabaseClient.instance = new DatabaseClient();
    }
    return DatabaseClient.instance;
  }

  /**
   * Synchronizes the database schema and generates TypeScript types
   */
  public async syncSchema(): Promise<void> {
    try {
      console.log('Syncing database schema...');
      await invoke('sync_schema');
      console.log('Database schema synced successfully');
    } catch (error) {
      console.error('Error syncing database schema:', error);
      throw error;
    }
  }

  /**
   * Queries a table with type-safe parameters
   */
  public async query<T = any>(params: QueryParams<T>): Promise<T[]> {
    try {
      console.log('Executing query:', params);
      const result = await invoke('query_table', {
        params: {
          table: params.table,
          columns: params.columns as string[],
          conditions: params.conditions?.map(([column, value]) => [column as string, value]),
          order_by: params.orderBy?.map(([column, asc]) => [column as string, asc]),
          limit: params.limit,
          offset: params.offset,
        },
      });
      console.log('Query result:', result);
      return result as T[];
    } catch (error) {
      console.error('Error executing query:', error);
      throw error;
    }
  }

  /**
   * Inserts a record into a table
   */
  public async insert<T>(params: InsertParams<T>): Promise<T> {
    return invoke('insert_into_table', {
      params: {
        table: params.table,
        value: params.value,
      },
    });
  }

  /**
   * Helper method to create a type-safe query builder
   */
  public table<T>(name: string) {
    return new TableQueryBuilder<T>(this, name);
  }
}

export class TableQueryBuilder<T> {
  private columns?: (keyof T)[];
  private conditions: Array<[keyof T, any]> = [];
  private orderByColumns: Array<[keyof T, boolean]> = [];
  private limitValue?: number;
  private offsetValue?: number;

  constructor(
    private client: DatabaseClient,
    private tableName: string,
  ) { }

  select(...columns: (keyof T)[]): this {
    this.columns = columns;
    return this;
  }

  where(column: keyof T, value: any): this {
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
      columns: this.columns,
      conditions: this.conditions,
      orderBy: this.orderByColumns,
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

// Export a singleton instance
export const db = DatabaseClient.getInstance(); 