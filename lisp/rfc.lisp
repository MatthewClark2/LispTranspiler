(define rfncall (lambda (f . args) (apply f (cons f args))))

(define map-aux (lambda (rec f alist out)
  (if (eqv (length alist) 0)
      (reverse out)
      (funcall rfncall 
	       rec
	       f
	       (cdr alist)
	       (cons (funcall f (car alist))
		     out)))))

(define map (lambda (f alist) (funcall rfncall map-aux f alist nil)))

(format (funcall map (lambda (x) (+ 5 x)) (list 1 2 3 4)))

