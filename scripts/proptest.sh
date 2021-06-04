#! /bin/bash
end=$((SECONDS+7200))

while [ $SECONDS -lt $end ]; do
    (cd puck && cargo test pt_ --features _test_fuzzing)
    if [[ x$? != x0 ]] ; then
        exit $?
    fi
done
