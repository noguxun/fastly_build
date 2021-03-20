
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

docker build -t noguxun/fastlybuild:latest -t noguxun/fastlybuild:001 . -f Dockerfile


###################################################

docker login
docker pull noguxun/fastlybuild:latest

docker run --network=host  --name fbuild noguxun/fastlybuild


###################################################

docker exec -it fbuild /bin/bash

docker image list

docker container ls -a

docker container stop $(docker container ls -aq)
docker container rm $(docker container ls -aq)

docker push noguxun/fastlybuild
docker pull noguxun/fastlybuild





