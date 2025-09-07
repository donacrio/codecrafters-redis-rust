echo -e "PING" | redis-cli &
echo -e "PING\nPING" | redis-cli &
echo -e "PING\nPING\nPING" | redis-cli &

wait