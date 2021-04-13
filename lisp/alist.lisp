(define new-list (lambda (. args) args))
(define min-list (lambda (x . args) (cons x args)))

(format (funcall new-list))
(format (funcall new-list 1))
(format (apply new-list (list)))
(format (funcall min-list ))
(format (funcall min-list 1))
(format (funcall min-list 1 2))

