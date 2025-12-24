#!/bin/bash
cargo build -p cli --release

sudo rm -f -R .debpkg

sudo mkdir -p .debpkg/DEBIAN/
sudo mkdir -p .debpkg/etc/yave
sudo mkdir -p .debpkg/usr/lib/yave
sudo mkdir -p .debpkg/var/lib/yave
sudo mkdir -p .debpkg/usr/bin

echo '#!/bin/bash' | sudo tee -a .debpkg/usr/lib/yave/netdevup
echo 'yave-cli netdev --name $YAVE_NAME --ifname $1 up' | sudo tee -a .debpkg/usr/lib/yave/netdevup
sudo chmod a+x .debpkg/usr/lib/yave/netdevup

echo '#!/bin/bash' | sudo tee -a .debpkg/usr/lib/yave/netdevdown
echo 'yave-cli netdev --name $YAVE_NAME --ifname $1 up' | sudo tee -a .debpkg/usr/lib/yave/netdevdown
sudo chmod a+x .debpkg/usr/lib/yave/netdevdown

echo '#!/bin/bash' | sudo tee -a .debpkg/DEBIAN/postrm
echo 'echo "Удалить все данные в /var/lib/yave? [y/N]"' | sudo tee -a .debpkg/DEBIAN/postrm
echo 'read ans' | sudo tee -a .debpkg/DEBIAN/postrm
echo 'if [[ "$ans" =~ ^[Yy]$ ]]; then' | sudo tee -a .debpkg/DEBIAN/postrm
echo '    rm -rf /var/lib/yave' | sudo tee -a .debpkg/DEBIAN/postrm
echo 'fi' | sudo tee -a .debpkg/DEBIAN/postrm
echo 'PACKAGE_GROUP="yave"' | sudo tee -a .debpkg/DEBIAN/postrm
echo 'groupdel "$PACKAGE_GROUP"' | sudo tee -a .debpkg/DEBIAN/postrm
echo 'rm -rf /run/yave' | sudo tee -a .debpkg/DEBIAN/postrm
sudo chmod a+x .debpkg/DEBIAN/postrm


echo '#!/bin/bash' | sudo tee -a .debpkg/DEBIAN/postinst
echo 'set -e' | sudo tee -a .debpkg/DEBIAN/postinst
echo 'PACKAGE_GROUP="yave"' | sudo tee -a .debpkg/DEBIAN/postinst
echo 'get_min_gid() {' | sudo tee -a .debpkg/DEBIAN/postinst
echo '    local min_gid=100' | sudo tee -a .debpkg/DEBIAN/postinst
echo '    while getent group "$min_gid" >/dev/null 2>&1; do' | sudo tee -a .debpkg/DEBIAN/postinst
echo '        min_gid=$((min_gid+1))' | sudo tee -a .debpkg/DEBIAN/postinst
echo '    done' | sudo tee -a .debpkg/DEBIAN/postinst
echo '    echo "$min_gid"' | sudo tee -a .debpkg/DEBIAN/postinst
echo '}' | sudo tee -a .debpkg/DEBIAN/postinst
echo 'if ! getent group "$PACKAGE_GROUP" >/dev/null 2>&1; then' | sudo tee -a .debpkg/DEBIAN/postinst
echo '    GID=$(get_min_gid)' | sudo tee -a .debpkg/DEBIAN/postinst
echo '    echo "Создаём группу $PACKAGE_GROUP с GID=$GID..."' | sudo tee -a .debpkg/DEBIAN/postinst
echo '    groupadd -g "$GID" "$PACKAGE_GROUP"' | sudo tee -a .debpkg/DEBIAN/postinst
echo 'fi' | sudo tee -a .debpkg/DEBIAN/postinst
echo 'exit 0' | sudo tee -a .debpkg/DEBIAN/postinst
sudo chmod a+x .debpkg/DEBIAN/postinst


sudo install ./default_config.yaml .debpkg/etc/yave/config.yaml

sudo install ./target/release/cli .debpkg/usr/bin/yave-cli

sudo install ./deb_package .debpkg/DEBIAN/control

echo 'Hi' | sudo tee -a .debpkg/var/lib/yave/.keep

sudo dpkg-deb --build .debpkg
