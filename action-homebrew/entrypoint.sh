#!/bin/sh
set -euxo pipefail

#declare variables
binaryUrl="https://github.com/WeareJH/wf2/releases/latest/download/wf2"
githubToken=$HOMEBREW_GITHUB_TOKEN

#setup git user
git config --global user.name "wf2 bot"
git config --global user.email "wf2@wearejh.com"

#download binary
echo "Downloading Binary file from $binaryUrl"
curl -s -o /binary $binaryUrl

#store hash
hash=$(shasum -a 256 /binary | awk '{print $1}')
echo "Binary hash is : $hash"

#store github fingerprint so that we can clone without interaction
mkdir ~/.ssh && touch ~/.ssh/known_hosts
ssh-keyscan github.com 2> /dev/null >> ~/.ssh/known_hosts

#store latest tag name
echo "Cloning wf2 repo..."
rm -rf wf2 && git clone --quiet https://$githubToken@github.com/WeareJH/wf2.git
cd wf2
tagName=$(git describe --tags `git rev-list --tags --max-count=1`)
cd /
echo "Latest tag is : $tagName"

#clone homebrew tools repository
echo "Cloning homebrew-tools repo..."
rm -rf homebrew-tools && git clone --quiet https://$githubToken@github.com/WeareJH/homebrew-tools.git
cd homebrew-tools && git pull origin master

#update hash and version number
echo "Updating hash and tag name..."
sed -i "s/sha256.*/sha256 '$hash'/g" wf2.rb
sed -i "s/version.*/version '$tagName'/g" wf2.rb
sed -i "s#url.*#url 'https://github.com/WeareJH/wf2/releases/download/$tagName/wf2'#g" wf2.rb

#commit and force push to homebrew repo
echo "Adding commit and pushing..."
git add wf2.rb
git commit -m "update to version $tagName"
git push --force

echo "Done!"
