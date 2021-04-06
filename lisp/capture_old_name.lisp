(define x 10)

(define x (lambda () x))

(format (funcall x))

