#!/usr/bin/env bash
command=$1
stateFile=stateFile

function helpFunc() {
    echo "Node manager"
    echo "  help        Provide help;"
    echo "  kill        Kill nodes, request a number of nodes to kill;"
    echo "  launch      Launch nodes, request a number of nodes to launch;"
    echo "  list        Return list of currently working nodes;"
}

function listFunc() {
    if [ -s stateFile ] ; 
    then
        currentCountOfNodes=$(head -n 1 $stateFile)
        echo "Now launched $currentCountOfNodes nodes"
    else
        echo "There isn't launched nodes"
    fi 
}

function launchFunc() {
    echo "Enter count of nodes"
    read count
    re='^[0-9]+$'
    if ! [[ $count =~ $re ]] ; then
    echo "error: Not a number" >&2; exit 
    fi

    echo "Enter contract address"
    read contract

    #run master node
    currentDir=$(pwd)
    mainNodeComand="cd $currentDir && cd ../node && cargo run --release -- -a 0.0.0.0:3000 --rpc-url http://localhost:8545 --contract '$contract'"
    osascript -e "tell app \"Terminal\"
        do script \"${mainNodeComand};\"
    end tell"
    sleep 2
    for ((i=1; i<$count; i++));
    do
        nodeComand="cd $currentDir && cd ../node && cargo run --release -- -a 0.0.0.0:300$i --peer 127.0.0.1:3000 --rpc-url http://localhost:8545 --contract '$contract'"
        osascript -e "tell app \"Terminal\"
            do script \"${nodeComand};\"
        end tell"
        echo $((i+1)) > stateFile
    done
    echo "Launched"
}

function killport() { 
    lsof -t -i tcp:$1 | xargs kill -9 
}

function killFunc() {
    if [ -s stateFile ] ; 
    then
        currentCountOfNodes=$(head -n 1 $stateFile)
        echo "Now launched  $currentCountOfNodes nodes"

        echo "How many nodes you want to kill?"
        read countToKill
        re='^[0-9]+$'
        if ! [[ $countToKill =~ $re ]] ; then
        echo "error: Not a number" >&2; exit 
        fi
        
        for ((i=$currentCountOfNodes; i>$((currentCountOfNodes-countToKill)); i--));
        do  
            killport "300$((i-1))"
            echo $((i-1)) > stateFile
        done
        echo "Killed"
    else
        echo "There isn't launched nodes"
    fi 
}

case $command in
    "help" )
        helpFunc ;;
    "launch" )
        launchFunc ;;
    "list" )
        listFunc ;;
    "kill" )
        killFunc ;;
    *)
        helpFunc ;;
esac