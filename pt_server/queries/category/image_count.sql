SELECT 
   COUNT(*) 
FROM 
   image_category ic
WHERE
   ic.category_id = $1;