#!/usr/bin/env bash
set -e
cd "$(dirname "${BASH_SOURCE[0]}")"

sudo apt update
sudo apt upgrade -y

# some of these might be able to be trimmed now that we're using llvm
sudo apt install -y build-essential pkg-config libssl-dev software-properties-common

sudo apt install -y unzip # for the gazetteer file

# install llvm so we can link with lld
# it takes less memory so builds can then be done right on the 1GB server
# https://apt.llvm.org/
wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key|sudo apt-key add
sudo add-apt-repository 'deb http://apt.llvm.org/focal/ llvm-toolchain-focal-14 main' # focal is ubuntu 20.04 (lts)
sudo apt-get -y install clang-14 lldb-14 lld-14 # llvm 14 released march 2022
ln -s /usr/bin/clang-14 /usr/bin/clang
ln -s /usr/bin/lld-14 /usr/bin/lld

# dvc - data version control, for images and stuff in our repository
sudo wget \
       https://dvc.org/deb/dvc.list \
       -O /etc/apt/sources.list.d/dvc.list
wget -qO - https://dvc.org/deb/iterative.asc | gpg --dearmor > packages.iterative.gpg
sudo install -o root -g root -m 644 packages.iterative.gpg /etc/apt/trusted.gpg.d/
rm -f packages.iterative.gpg
sudo apt update
sudo apt install -y dvc

./get_rust.sh

# use this method so it's installed for all users (nvm is only for one user)
# https://www.digitalocean.com/community/tutorials/how-to-install-node-js-on-ubuntu-20-04
# I *think* this will include updates because it's a PPA
curl -o- https://deb.nodesource.com/setup_24.x | bash # match version in frontend_test.yaml
sudo apt install -y nodejs
node ../frontend/prebuild.js # fail if the installed release predates the CVE fix

# ssl proxy / certbot / let's encrypt stuff
sudo apt install -y nginx

sudo snap install core; sudo snap refresh core
sudo snap install --classic certbot
sudo ln -s /snap/bin/certbot /usr/bin/certbot

# https://certbot.eff.org/instructions?ws=nginx&os=ubuntufocal
sudo snap set certbot trust-plugin-with-root=ok
sudo snap install certbot-dns-digitalocean
