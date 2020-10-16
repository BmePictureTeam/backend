UPDATE category
SET category_name = $2
WHERE
	id = $1;