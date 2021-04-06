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
