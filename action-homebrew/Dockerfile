FROM alpine:3.9

LABEL "com.github.actions.name"="Update Homebrew"
LABEL "com.github.actions.description"="Updates homebrew binary hash and version number"
LABEL "com.github.actions.icon"="package"
LABEL "com.github.actions.color"="purple"

LABEL "maintainer"="Diego Cabrejas <diego@wearejh.com>"

RUN apk add --no-cache git
RUN apk add --no-cache openssh
RUN apk add --no-cache curl
RUN apk add --no-cache perl-utils

ADD entrypoint.sh /entrypoint.sh
RUN chmod +x entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
