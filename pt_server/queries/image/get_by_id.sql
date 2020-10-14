SELECT created,
	description,
	id,
	app_user_id,
	title,
	upload_date
FROM image i
WHERE i.id = $1;