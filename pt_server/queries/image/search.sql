SELECT
	*
FROM
	image i
WHERE
	i.title % $1
	OR i.description % $1
ORDER BY
	SIMILARITY(i.title, $1) DESC
OFFSET $2
LIMIT $3;