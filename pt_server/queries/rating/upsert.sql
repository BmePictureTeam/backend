INSERT
	INTO
	rating(app_user_id, image_id, rating)
VALUES ($1, $2, $3) 
ON CONFLICT (app_user_id, image_id)
DO UPDATE
SET
	rating = $3;