# ./run.sh list - 列出可用 .cir 範例
# ./run.sh cir <file.cir> - 執行 SPICE netlist
# ./run.sh svg [file] - 匯出電路圖（可指定檔名）
# ./run.sh cir circuits/divider.cir --svg circuits/divider.svg
set -x
./run.sh list 

./run.sh cir circuits/divider.cir --svg circuits/divider.svg
./run.sh cir circuits/rc_charge.cir --svg circuits/rc_charge.svg
./run.sh cir circuits/rc_lowpass.cir --svg circuits/rc_lowpass.svg
./run.sh cir circuits/rectifier.cir --svg circuits/rectifier.svg
./run.sh cir circuits/rlc_bandpass.cir --svg circuits/rlc_bandpass.svg
./run.sh cir circuits/wheatstone.cir --svg circuits/wheatstone.svg
