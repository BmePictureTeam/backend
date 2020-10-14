SELECT
	created, description, id, title, upload_date
FROM
	image i
WHERE
	i.app_user_id = $1;