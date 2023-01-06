#!/usr/bin/env python

import sys

argc = len(sys.argv)

gpios=[]
pulses=[]

with open(sys.argv[1], 'r') as f:
   edges=[]
   num_bits = 0
   for line in f:
      if len(line) > 1 and line[0] != '#':
         fields = line.split()
         gpio = int(fields[0])
         gpios.append(gpio)
         levels = int(fields[1], 2)
         edges.append([1<<gpio, levels])
         if len(fields[1]) > num_bits:
            num_bits = len(fields[1])

bit_time = int(sys.argv[2])

for bit in range(num_bits-1, -1, -1):
   on = 0
   off = 0
   for e in edges:
      if (1<<bit) & e[1]:
         on |= e[0]
      else:
         off |= e[0]
   pulses.append((on, off, bit_time))


import pigpio

pi = pigpio.pi()
if not pi.connected:
  exit()

for g in gpios:
  pi.set_mode(g, pigpio.OUTPUT)

wf = []
for p in pulses:
  wf.append(pigpio.pulse(p[0], p[1], p[2]))

pi.wave_clear()
pi.wave_add_generic(wf)

wid = pi.wave_create()
if wid >= 0:
  if int(sys.argv[3]) == 0: 
    pi.wave_send_repeat(wid)
  else:
    for i in range(0,int(sys.argv[3])):
      pi.wave_send_once(wid)
  sys.stdin.readline()
  pi.wave_tx_stop()
  pi.wave_delete(wid)

for g in gpios:
  pi.write(g, 0)

pi.stop()
