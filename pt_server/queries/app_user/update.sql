UPDATE
	app_user
SET
	email = $2,
	password_hash = $3,
	is_admin = $4
WHERE
	app_user.id = $1;