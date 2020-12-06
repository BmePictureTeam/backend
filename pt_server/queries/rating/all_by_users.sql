SELECT
	CAST (AVG(rating) AS FLOAT) AS average_rating,
	au.email AS email
FROM
	rating r
INNER JOIN image i ON
	r.image_id = i.id
INNER JOIN app_user au ON
	au.id = i.app_user_id
GROUP BY email
ORDER BY average_rating;