(define alist (list 1 2 3))
(define alist-tail (cdr alist))

(format "alist" alist)

(format "(car alist)" (car alist))
(format "(cdr alist)" (cdr alist))
(format "(reverse alist)" (reverse alist))

(define alist2 (cons 0 alist))
(format "alist2" alist2)
(format "(length alist2)" (length alist2))

(format (+ (length alist2) (length alist)))
