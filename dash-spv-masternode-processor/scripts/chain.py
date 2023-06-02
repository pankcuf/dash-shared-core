from enum import Enum, unique
@unique
class Chain(Enum):
    MAINNET = "mainnet", "insight.dashevo.org"
    TESTNET = "testnet", "insight.testnet.networks.dash.org:3002"
    SCREWDRIVER = "screwdriver", "insight.screwdriver.networks.dash.org:3002"
    ABSINTHE = "absinthe", "insight.absinthe.networks.dash.org:3002"

    def __new__(cls, name, api_base):
        obj = object.__new__(cls)
        obj._value_ = name
        obj.api_base = api_base
        return obj

    @property
    def base(self):
        return self.api_base

    @classmethod
    def from_string(cls, chain_str):
        for member in cls:
            if member.value == chain_str.lower():
                return member
        raise ValueError(f"{chain_str} is not a valid Chain")

