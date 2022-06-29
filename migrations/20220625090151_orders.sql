CREATE TABLE orders (
  "id" bigint PRIMARY KEY,
  "card_id" int NOT NULL,
  "price" int NOT NULL,
  "side" smallint NOT NULL,
  "status" smallint NOT NULL,
  "trader_id" bigint NOT NULL REFERENCES traders(id),
  "created_at" timestamp WITH time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);