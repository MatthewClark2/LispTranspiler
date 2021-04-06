;; Nested calls get unwrapped.
;(format "hello" (get-world))

;; Condition in call is unwrapped.
;(format (if some-condition :yes :no))

;; Call in condition is fine.
;(if asymbol (say-hello) (say-goodbye))

;; Some nested conditions are not unwrapped.
(if something (if something-else :is-both :is-first) :is-not-first)

;; Others are.
;(if (if sth :yes :no) (if-yes) (if-no))

