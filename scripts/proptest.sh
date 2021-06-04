#! /bin/bash

$exit_code=0

end=$((SECONDS+7200))

while [ $SECONDS -lt $end ]; do
    (cd puck && cargo test pt_ --release --features _test_fuzzing "$@")
    if [[ x$? != x0 ]] ; then
        exit_code=1
        break
    fi
done

exit $exit_code
