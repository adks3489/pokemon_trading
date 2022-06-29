CREATE TABLE traders (
  "id" bigint GENERATED always AS IDENTITY PRIMARY KEY,
  "created_at" timestamp WITH time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);