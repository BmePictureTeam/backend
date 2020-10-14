INSERT
	INTO
	image (app_user_id, upload_date, title, description)
VALUES ($1, $2, $3, $4) RETURNING id;