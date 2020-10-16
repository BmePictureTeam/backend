INSERT INTO category (category_name)
VALUES ($1)
RETURNING category.id;