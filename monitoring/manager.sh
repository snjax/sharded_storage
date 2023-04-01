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
        sed 1d $stateFile | while read -r line
        do
            kill 3486 "$line"
        done
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

    tee stateFile <<<$count
    for ((i=0; i<$count; i++));
    do
        open -a TextEdit stateFile --new & 
        _pid=$! >> stateFile
        echo "$_pid"
    done
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

        killedCount=1;
        sed 1d $stateFile | while read -r line
        do  
            if [ $killedCount \> $countToKill ] ; then
                newCountOfNodes=$((currentCountOfNodes - countToKill))
                echo "Now launched $newCountOfNodes nodes"
                exit
            fi

            echo $line
            #kill $line
            killedCount=$killedCount+1
        done
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