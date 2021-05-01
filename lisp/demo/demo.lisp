;; Once again, we have comments!

;; Everything compiles now!

;; Variables can be redefined.
(define x (list :a :b :c))
(format "x: " x)

(define y (cdr x)) ; (:b :c)

(define x 10)
(format "x: " x)
(format "y: " y)

;; We can have conditions!
(if (< 2 5) (format "2 < 5") (format "5 < 2"))

;; They can also be used as values.
(format "Are empty lists false? " (if (list) "no" "yes"))

;; Function fanciness.

;; Lambdas have no natural recursion, so it needs to be packaged like so.
(define rfncall (lambda (f . args) (apply f (cons f args))))

;; Utility function that retains accumulator and recursive form.
(define map-aux (lambda (rec f alist out)
  (if (eqv (length alist) 0)
      (reverse out)
      (funcall rfncall 
	       rec
	       f
	       (cdr alist)
	       (cons (funcall f (car alist))
		     out)))))

(define map (lambda (f alist)
	      (funcall rfncall map-aux f alist nil)))

(format (funcall map (lambda (x) (+ 5 x)) (list 1 2 3 4)))

