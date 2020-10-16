SELECT
	c.id,
	c.created,
	c.category_name
FROM
	category c
INNER JOIN image_category ic ON
	ic.image_id = $1;