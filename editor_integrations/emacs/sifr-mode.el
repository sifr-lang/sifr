;;; sifr-mode.el --- Sifr filetype and LSP integration -*- lexical-binding: t; -*-

(require 'eglot)

(define-derived-mode sifr-mode prog-mode "Sifr"
  "Major mode for Sifr source files."
  (setq-local comment-start "#")
  (setq-local comment-end ""))

(add-to-list 'auto-mode-alist '("\\.sifr\\'" . sifr-mode))
(add-to-list 'eglot-server-programs '(sifr-mode . ("sifr" "lsp" "--stdio")))

(provide 'sifr-mode)

;;; sifr-mode.el ends here
