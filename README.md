# esp32c3-current-monitor

| Breadboard Overview | Demo GIF |
| ------------------- | -------- |
| <img src="https://github.com/user-attachments/assets/18972853-5c9f-4027-b14e-49f30249fab9" style="width:450px;" /> | <img src="https://github.com/user-attachments/assets/26fecbc9-8c3e-4ae9-aef0-dbb88c79499a" style="width:auto;" /> |


```shell
nix-shell
cargo run --release
# wait for flashing to complete...
 
# Press GPIO0 (BOOT BTN) to cycle averaging mode between 1,4,16,64,128,256,512,1024 samples. Reset to go to 1x.
  
# current uA = current shunt mV * 100 
# 8mV == 800uA == 0.8mA
 
# serial output, time, current shunt voltage, bus voltage.
Init!
2669876 us, 0 mV, 0 mV
2802594 us, 0 mV, 0 mV
2935287 us, 8 mV, 0 mV
3069009 us, 0 mV, 0 mV
3201762 us, 0 mV, 0 mV
3334489 us, 8 mV, 0 mV
3468201 us, 0 mV, 0 mV
3600975 us, 0 mV, 0 mV
3733712 us, 8 mV, 0 mV
3867438 us, 0 mV, 0 mV
4000235 us, 0 mV, 0 mV
4132990 us, 8 mV, 0 mV
4266799 us, 0 mV, 0 mV
4399605 us, 0 mV, 0 mV
```


