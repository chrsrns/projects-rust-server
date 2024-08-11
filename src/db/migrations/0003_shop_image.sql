CREATE TABLE IF NOT EXISTS shop_image (
    id SERIAL PRIMARY KEY,
    shop_item_id INT NOT NULL,
    tooltip VARCHAR NOT NULL,
    img_link VARCHAR NOT NULL,
    CONSTRAINT fk_shop_item FOREIGN KEY (shop_item_id) REFERENCES shop_item (id)
);
