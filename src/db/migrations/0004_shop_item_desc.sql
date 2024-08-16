CREATE TABLE IF NOT EXISTS shop_item_desc (
    id SERIAL PRIMARY KEY,
    shop_item_id INT NOT NULL,
    content VARCHAR NOT NULL,
    CONSTRAINT fk_shop_item FOREIGN KEY (shop_item_id) REFERENCES shop_item (id)
);
