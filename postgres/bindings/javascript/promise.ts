import {
  DatabasePromise,
  NativeDatabase,
  SqliteError,
  DatabaseOpts,
} from "@tursodatabase/database-common";
import { Database as NativeDB } from "#index";

class Database extends DatabasePromise {
  constructor(path: string, opts: DatabaseOpts = {}) {
    super(new NativeDB(path, opts) as unknown as NativeDatabase);
  }
}

async function connect(
  path: string,
  opts: DatabaseOpts = {}
): Promise<Database> {
  const db = new Database(path, opts);
  await db.connect();
  return db;
}

export { connect, Database, SqliteError };
