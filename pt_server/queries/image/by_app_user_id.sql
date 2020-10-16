SELECT
	*
FROM
	image i
WHERE
	i.app_user_id = $1;