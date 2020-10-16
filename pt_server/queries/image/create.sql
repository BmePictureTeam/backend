INSERT
	INTO
	image (app_user_id, title, description)
VALUES ($1, $2, $3) RETURNING id;