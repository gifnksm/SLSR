#!/bin/bash -ex

# download the puzzles from the Internet

ETCDIR=$(cd $(dirname $0) && pwd -P)
BASEDIR=$(cd ${ETCDIR}/.. && pwd -P)

download_janko() {
    local NUM="$1"
    local BASEURL="http://www.janko.at/Raetsel/Slitherlink"

    curl -s "${BASEURL}/${NUM}.a.htm" |
        perl -ne 'if (s/^problem\n// .. s/^solution\n|^unit \w+\n//) { s/ *//g; print; }'
}

download_java() {
    local NUM="$1"
    local BASEURL="http://www.pro.or.jp/~fuji/java/puzzle/numline"

    curl -s "${BASEURL}/${NUM}.data" |
        perl -ne 'if (s/^problem\n// .. s/^end\n//) { s/ *//g; print; }'
}

main() {
    local I
    local JANKO_DIR="${BASEDIR}/puzzle/janko"
    mkdir -pv "${JANKO_DIR}"
    for I in {1..930}; do
        local NUM="$(printf "%03d" "${I}")"
        local FILE="${JANKO_DIR}/${NUM}.txt"
        if ! [ -s "${FILE}" ]; then
            download_janko "${NUM}" > "${FILE}"
            sleep 1
        fi
    done

    local JAVA_DIR="${BASEDIR}/puzzle/java"
    local TYPE=book1
    mkdir -pv "${JAVA_DIR}/${TYPE}"
    for I in {1..85}; do
        local NUM="$(printf "%03d" "${I}")"
        local FILE="${JAVA_DIR}/${TYPE}/${NUM}.txt"
        if ! [ -s "${FILE}" ]; then
            download_java "${TYPE}/${NUM}" > "${FILE}"
            sleep 1
        fi
    done

    local TYPE=misc
    mkdir -pv "${JAVA_DIR}/${TYPE}"
    for I in {1..34}; do
        local NUM="$(printf "%03d" "${I}")"
        local FILE="${JAVA_DIR}/${TYPE}/${NUM}.txt"
        if ! [ -s "${FILE}" ]; then
            download_java "${NUM}" > "${FILE}"
            sleep 1
        fi
    done
}

main
