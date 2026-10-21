#!/bin/bash
set -euo pipefail
test -f /run/lab/password
test -f /run/lab/id_ed25519.pub
password=$(cat /run/lab/password)
printf 'tester:%s\n' "$password" | chpasswd
install -d -m 700 -o tester -g tester /home/tester/.ssh
install -m 600 -o tester -g tester /run/lab/id_ed25519.pub /home/tester/.ssh/authorized_keys
ssh-keygen -A
printf '\nPasswordAuthentication yes\nPermitRootLogin no\n' >> /etc/ssh/sshd_config
openssl req -x509 -newkey rsa:2048 -nodes -days 2 -keyout /etc/xrdp/key.pem -out /etc/xrdp/cert.pem -subj '/CN=localhost' -addext 'subjectAltName=DNS:localhost,IP:127.0.0.1' >/dev/null 2>&1
chown xrdp:xrdp /etc/xrdp/key.pem /etc/xrdp/cert.pem
printf '%s' "$password" | tigervncpasswd -f > /home/tester/.vnc-password
chown tester:tester /home/tester/.vnc-password
chmod 600 /home/tester/.vnc-password
mkdir -p /home/tester/fixtures/subdir
printf 'Texto de prueba UTF-8: áéíóú\n' > /home/tester/fixtures/unicode.txt
dd if=/dev/zero of=/home/tester/fixtures/large.bin bs=1M count=16 status=none
chown -R tester:tester /home/tester/fixtures
unset password
runuser -u tester -- /usr/bin/Xtigervnc :1 -rfbport 5901 -localhost no -SecurityTypes VncAuth -PasswordFile /home/tester/.vnc-password -geometry 1280x800 -depth 24 >/tmp/vnc.log 2>&1 &
runuser -u tester -- env DISPLAY=:1 fluxbox >/tmp/fluxbox.log 2>&1 &
/usr/sbin/xrdp-sesman --nodaemon >/tmp/sesman.log 2>&1 &
/usr/sbin/xrdp --nodaemon >/tmp/xrdp.log 2>&1 &
exec /usr/sbin/sshd -D -e
