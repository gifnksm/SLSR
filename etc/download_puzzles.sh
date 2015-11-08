#!/bin/bash -ex

# download the puzzles from the Internet

ETCDIR=$(readlink -f $(dirname $0))
BASEDIR=$(readlink -f ${ETCDIR}/..)

download_janko() {
    local NUM=$1
    local BASEURL="http://www.janko.at/Raetsel/Slitherlink"

    curl -s "${BASEURL}/${NUM}.a.htm" |
        sed -n -e '1,/^problem/!p' |
        sed -n -e '/^solution\|^unit/,$!p' |
        sed 's/ //g'
}

download_java() {
    local NUM=$1
    local BASEURL="http://www.pro.or.jp/~fuji/java/puzzle/numline"

    curl -s "${BASEURL}/${NUM}.data" |
        sed -n -e '1,/problem/!p' |
        sed -n -e '/end/,$!p' |
        sed 's/ //g'
}

main() {
    local JANKO_DIR="${BASEDIR}/puzzle/janko"
    mkdir -pv "${JANKO_DIR}"
    for NUM in {001..930}; do
        local FILE="${JANKO_DIR}/${NUM}.txt"
        if ! [ -s "${FILE}" ]; then
            download_janko "${NUM}" > "${FILE}"
            sleep 1
        fi
    done

    local JAVA_DIR="${BASEDIR}/puzzle/java"
    local TYPE=book1
    mkdir -pv "${JAVA_DIR}/${TYPE}"
    for NUM in {001..085}; do
        local FILE="${JAVA_DIR}/${TYPE}/${NUM}.txt"
        if ! [ -s "${FILE}" ]; then
            download_java "${TYPE}/${NUM}" > "${FILE}"
            sleep 1
        fi
    done

    local TYPE=misc
    mkdir -pv "${JAVA_DIR}/${TYPE}"
    for NUM in {001..034}; do
        local FILE="${JAVA_DIR}/${TYPE}/${NUM}.txt"
        if ! [ -s "${FILE}" ]; then
            download_java "${NUM}" > "${FILE}"
            sleep 1
        fi
    done
}

main
