(format (if (< 5 6) "5 < 6" (+ "crash")))
(format (if (> 5 6) (+ "crash") "!(5 < 6)"))
(format (if #t "true" "false") "branch executed")
