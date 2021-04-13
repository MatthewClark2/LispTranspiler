(define ack-aux
    (lambda (a m n)
      (if (eqv m 0) (+ n 1)
          (if (eqv n 0) (funcall a a (- m 1) 1)
              (funcall a a (- m 1) (funcall a a m (- n 1)))))))

(define ack (lambda (m n) (funcall ack-aux ack-aux m n)))

(format (funcall ack 2 2))
