{ ... }: {
  programs.neomutt = {
    enable = true;
    vimKeys = true;
  };

  xdg.configFile."neomutt/neomuttrc".text = ''
    set editor = "nvim"
    set pager = "nvim -R -c 'set ft=mail'"
    set sort = reverse-date
    set ssl_force_tls = yes
    set imap_qresync = yes
    set header_cache = "~/.cache/neomutt/headers"
    set message_cachedir = "~/.cache/neomutt/bodies"

    source ~/.config/neomutt/account
  '';
}
