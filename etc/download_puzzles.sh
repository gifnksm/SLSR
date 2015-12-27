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

download_nikoli() {
    local NUM="$1"
    local BASEURL="http://www.nikoli.com/nfp"

    curl -s "${BASEURL}/sl-${NUM}.nfp" |
        sed 's/&/\'$'\n''/g' | sed -n 's/^dataQuestion=//p' | ${ETCDIR}/urldecode.py |
        sed 's/+//g'
}

main() {
    local I
    local NUMS

    local JANKO_DIR="${BASEDIR}/puzzle/janko"
    mkdir -pv "${JANKO_DIR}"
    if [ -z "${ONLY_TOP10}" ]; then
        NUMS=($(seq 1 930))
    else
        NUMS=(709 720 100 840 188 192 660 59 190)
    fi
    for I in "${NUMS[@]}"; do
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
    if [ -z "${ONLY_TOP10}" ]; then
        NUMS=($(seq 1 85))
    else
        NUMS=()
    fi
    for I in "${NUMS[@]}"; do
        local NUM="$(printf "%03d" "${I}")"
        local FILE="${JAVA_DIR}/${TYPE}/${NUM}.txt"
        if ! [ -s "${FILE}" ]; then
            download_java "${TYPE}/${NUM}" > "${FILE}"
            sleep 1
        fi
    done

    local TYPE=misc
    mkdir -pv "${JAVA_DIR}/${TYPE}"
    if [ -z "${ONLY_TOP10}" ]; then
        NUMS=($(seq 1 34))
    else
        NUMS=(34)
    fi
    for I in "${NUMS[@]}"; do
        local NUM="$(printf "%03d" "${I}")"
        local FILE="${JAVA_DIR}/${TYPE}/${NUM}.txt"
        if ! [ -s "${FILE}" ]; then
            download_java "${NUM}" > "${FILE}"
            sleep 1
        fi
    done

    local NIKOLI_DIR="${BASEDIR}/puzzle/nikoli"
    mkdir -pv "${NIKOLI_DIR}"
    if [ -z "${ONLY_TOP10}" ]; then
        NUMS=($(seq 1 10))
    else
        NUMS=()
    fi
    for I in "${NUMS[@]}"; do
        local NUM="$(printf "%04d" "${I}")"
        local FILE="${NIKOLI_DIR}/${NUM}.txt"
        if ! [ -s "${FILE}" ]; then
            download_nikoli "${NUM}" > "${FILE}"
            sleep 1
        fi
    done
}

main
