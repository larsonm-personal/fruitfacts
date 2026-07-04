CREATE TABLE collection_categories (
  id INTEGER PRIMARY KEY NOT NULL,
  collection_id INTEGER NOT NULL,
  category TEXT NOT NULL,

  UNIQUE(collection_id, category)
);
