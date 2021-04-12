(define map-aux (lambda (rec f alist out)
  (if (eqv (length alist) 0)
      (reverse out)
      (funcall rec 
	       f
	       (cdr alist)
	       (cons (funcall f (car alist))
		     out)))))

(define map (lambda (f alist) (funcall map-aux map-aux f alist (list))))

(define nums (list 1 2 3))

(format (funcall map (lambda (x) (+ 1 x)) nums))

