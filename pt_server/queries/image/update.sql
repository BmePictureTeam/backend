UPDATE image
SET upload_date = $2,
	title = $3,
	description = $4
WHERE
	id = $1;