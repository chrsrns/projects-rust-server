CREATE TABLE IF NOT EXISTS app_user (
    id SERIAL PRIMARY KEY,
    username VARCHAR UNIQUE NOT NULL,
    upassword VARCHAR NOT NULL,
    email VARCHAR UNIQUE NOT NULL
);
CREATE TABLE IF NOT EXISTS shop_item (
    id SERIAL PRIMARY KEY,
    iname VARCHAR UNIQUE NOT NULL,
    img_link VARCHAR NOT NULL,
    price DECIMAL NOT NULL
);
