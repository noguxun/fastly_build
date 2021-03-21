
sudo apt-get update

sudo apt-get install \
    apt-transport-https \
    ca-certificates \
    curl \
    gnupg \
    lsb-release

curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg

echo \
  "deb [arch=amd64 signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu \
  $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

sudo apt-get update
sudo apt-get install docker-ce docker-ce-cli containerd.io

sudo systemctl status docker

sudo usermod -aG docker $USER

sudo reboot

docker container run hello-world


###################################################
#### dev

docker build -t noguxun/fastlybuild:latest . -f Dockerfile
docker run -p 127.0.0.1:8080:8080 -it --rm --name fbuild noguxun/fastlybuild
docker push noguxun/fastlybuild

###################################################

docker login
docker pull noguxun/fastlybuild:latest

docker run --network=host  --name fbuild noguxun/fastlybuild
docker run -p 127.0.0.1:8080:8080 -it --rm --name fbuild noguxun/fastlybuild




###################################################

docker exec -it fbuild /bin/bash

docker image list

docker container ls -a

docker container stop $(docker container ls -aq)
docker container rm $(docker container ls -aq)

docker push noguxun/fastlybuild
docker pull noguxun/fastlybuild

###################################################

https://rustwasm.xgu.tokyo/rust-crypto-wasm


