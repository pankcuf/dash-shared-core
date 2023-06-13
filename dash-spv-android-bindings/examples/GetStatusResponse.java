//package org.dashj.example;

public class GetStatusResponse {

    public static class Version {
        public int protocol;
        public int software;
        public String agent;

        public Version(int protocol, int software, String agent) {
            this.protocol = protocol;
            this.software = software;
            this.agent = agent;
        }
    }

    public static class Time {
        public long now;
        public int offset;
        public long median;

        public Time(long now, int offset, long median) {
            this.now = now;
            this.offset = offset;
            this.median = median;
        }
    }

    public static class Chain {
        public String name;
        public int headersCount;
        public int blocksCount;
        public byte[] bestBlockHash;
        public double difficulty;
        public byte[] chainWork;
        public boolean isSynced;
        public double syncProgress;

        // Constructor
        // ...
    }

    public static class Masternode {
        public int status;
        public byte[] proTxHash;
        public int posePenalty;
        public boolean isSynced;
        public double syncProgress;

        // Constructor
        // ...
    }

    public static class NetworkFee {
        public double relay;
        public double incremental;

        public NetworkFee(double relay, double incremental) {
            this.relay = relay;
            this.incremental = incremental;
        }
    }

    public static class Network {
        public int peersCount;
        public NetworkFee fee;

        public Network(int peersCount, NetworkFee fee) {
            this.peersCount = peersCount;
            this.fee = fee;
        }
    }

    public Version version;
    public Time time;
    public int status;
    public double syncProgress;
    public Chain chain;
    public Masternode masternode;
    public Network network;

    public GetStatusResponse(Version version, Time time, int status, double syncProgress, Chain chain, Masternode masternode, Network network) {
        this.version = version;
        this.time = time;
        this.status = status;
        this.syncProgress = syncProgress;
        this.chain = chain;
        this.masternode = masternode;
        this.network = network;
    }
}