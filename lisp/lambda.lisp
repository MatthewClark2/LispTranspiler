(define y :whatever)

(define f (lambda (x) (format x y)))

(apply f (list :one))
(funcall f :one)


