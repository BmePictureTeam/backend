INSERT
	INTO
	app_user (email, password_hash, is_admin)
VALUES($1, $2, $3) RETURNING id;