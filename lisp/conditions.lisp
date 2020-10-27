(define t #t)
(define f #f)

(format t #t)
(format f #f)

(format "(eqv)" (eqv))
(format "(eqv #t)" (eqv #t))
(format "(eqv #t #t)" (eqv #t #t))
(format "(eqv #t #t #f)" (eqv #t #t #f))

(format (if (< 5 6) "5 < 6" (+ "crash")))
(format (if (> 5 6) (+ "crash") "!(5 < 6)"))
