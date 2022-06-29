CREATE TABLE trades (
  "id" bigint GENERATED always AS IDENTITY PRIMARY KEY,
  "card_id" int NOT NULL,
  "price" int NOT NULL,
  "buyorder_id" bigint NOT NULL REFERENCES orders(id),
  "sellorder_id" bigint NOT NULL REFERENCES orders(id),
  "created_at" timestamp WITH time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);
