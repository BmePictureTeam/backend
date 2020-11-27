SELECT c.id,
	c.created,
	c.category_name
FROM category c
WHERE EXISTS (
		SELECT ic.category_id
		FROM image_category ic
		WHERE ic.image_id = $1
			AND ic.category_id = c.id
	);