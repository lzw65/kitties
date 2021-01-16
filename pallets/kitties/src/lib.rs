#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, EncodeLike};
use frame_support::dispatch::Vec;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::fmt::Debug,
    ensure,
    traits::{Currency, Randomness, ReservableCurrency},
    Parameter, StorageValue,
};
use frame_system::ensure_signed;
use sp_io::hashing::blake2_128;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, Bounded, MaybeDisplay, MaybeSerialize, Member},
    DispatchError,
};
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

//  Q4
#[derive(Encode, Decode, Clone)]
pub struct Kitty<AccountId, KittyIndex> {
    kitty_id: KittyIndex,
    dna: [u8; 16],
    owner: AccountId,
    parents: Option<(KittyIndex, KittyIndex)>,
    children: Option<Vec<KittyIndex>>,
    breed_partners: Option<Vec<KittyIndex>>,
}

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Randomness: Randomness<Self::Hash>;

    // Q2
    type KittyIndex: Parameter
        + Member
        + MaybeSerialize
        + Debug
        + Default
        + MaybeDisplay
        + AtLeast32BitUnsigned
        + Copy
        + Bounded
        + EncodeLike;

    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
}

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

decl_storage! {
    trait Store for Module<T: Trait> as KittiesModule {
        pub Kitties get(fn kitties): map hasher(blake2_128_concat) T::KittyIndex => Option<Kitty<T::AccountId, T::KittyIndex>>;
        pub KittiesCount get(fn kitties_count): T::KittyIndex;
        //pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) T::KittyIndex => Option<T::AccountId>;

        //Q3
        pub KittyList get(fn kitty_list): map hasher(blake2_128_concat) T::AccountId => Option<Vec<T::KittyIndex>>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        KittyIndex = <T as Trait>::KittyIndex,
    {
        Created(AccountId, KittyIndex),
        Transferred(AccountId, AccountId, KittyIndex),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Error names should be descriptive.
        KittiesCountOverflow,
        InvalidKittyId,
        RequireDifferentParent,
        NotOwner,
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        #[weight = 10_000]
        pub fn create(origin){
            let sender = ensure_signed(origin)?;
            let kitty_id = Self::next_kitty_id()?;
            let dna = Self::random_value(&sender);
            //Q6
            T::Currency::reserve(&sender, BalanceOf::<T>::from(500))?;
            let kitty = Kitty{
                kitty_id: kitty_id,
                dna: dna,
                owner: sender.clone(),
                parents: None,
                children: None,
                breed_partners: None
            };

            Self::insert_kitty(&sender, kitty_id, &kitty);
            Self::deposit_event(RawEvent::Created(sender, kitty_id));
        }

        #[weight = 10_000]
        pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex){
            let sender = ensure_signed(origin)?;
            // Q1
            // check if kitty_id's owner is sender
            if let Some(kitty) = Self::kitties(kitty_id){
                let owner = kitty.clone().owner;
                ensure!(owner.clone() == sender.clone(), Error::<T>::NotOwner);
                //Q6
                T::Currency::reserve(& to, BalanceOf::<T>::from(500))?;
                T::Currency::unreserve(& sender,  BalanceOf::<T>::from(500));

                //change kitty's ownner
                <Kitties<T>>::mutate(kitty_id, |k| {
                    let new_kitty = Kitty{
                        owner: to.clone(),
                        ..kitty
                    };
                    *k = Some(new_kitty);
                });

                // move kitty_id from sender's KittyList to To's KittyList
                if <KittyList<T>>::contains_key(&sender){
                    if let Some(kitty_list) = <KittyList<T>>::take(&sender){
                        let list: Vec<_> = kitty_list.clone().iter().filter(|kd|{*kd != &kitty_id}).map(|k|{*k}).collect();
                        <KittyList<T>>::insert(&sender, list);
                    };
                }
                if <KittyList<T>>::contains_key(&to){
                    if let Some(kitty_list) = <KittyList<T>>::take(&to){
                        let mut list: Vec<_> = kitty_list.to_vec();
                        list.push(kitty_id);
                        <KittyList<T>>::insert(&to, list);
                    };
                }else{
                    let mut list: Vec<T::KittyIndex> = Vec::new();
                    list.push(kitty_id);
                    <KittyList<T>>::insert(&to, list);
                }

                Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));
            }
        }
        #[weight = 10_000]
        pub fn breed(origin, kitty_id1: T::KittyIndex, kitty_id2: T::KittyIndex){
			let sender = ensure_signed(origin)?;
			//Q5
            T::Currency::reserve(&sender, BalanceOf::<T>::from(500))?;
            let new_kitty_id = Self::do_bread(&sender, kitty_id1, kitty_id2)?;
            Self::deposit_event(RawEvent::Created(sender, new_kitty_id));
        }

    }
}

impl<T: Trait> Module<T> {
    fn insert_kitty(
        owner: &T::AccountId,
        kitty_id: T::KittyIndex,
        kitty: &Kitty<T::AccountId, T::KittyIndex>,
    ) {
        Kitties::<T>::insert(kitty_id, kitty);
        KittiesCount::<T>::put(kitty_id + T::KittyIndex::from(1));

        if KittyList::<T>::contains_key(owner) {
            let mut k_list = <KittyList<T>>::take(owner).unwrap();
            k_list.push(kitty_id);
            KittyList::<T>::insert(owner, k_list);
        } else {
            let mut list: Vec<T::KittyIndex> = Vec::new();
            list.push(kitty_id);
            KittyList::<T>::insert(owner, list);
        }
    }

    fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError> {
        let kitty_id = Self::kitties_count();
        if kitty_id == T::KittyIndex::max_value() {
            return Err(Error::<T>::KittiesCountOverflow.into());
        }
        Ok(kitty_id)
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            T::Randomness::random_seed(),
            &sender,
            <frame_system::Module<T>>::extrinsic_index(),
        );
        payload.using_encoded(blake2_128)
    }

    fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
        (selector & dna1) | (!selector & dna2)
    }

    fn update_kitty(kitty_id: T::KittyIndex, child_id: T::KittyIndex, partner_id: T::KittyIndex) {
        if <Kitties<T>>::contains_key(&kitty_id) {
            let mut kitty = <Kitties<T>>::take(&kitty_id).unwrap();

            if let Some(mut children) = kitty.children {
                children.push(child_id);
                kitty.children = Some(children);
            } else {
                let mut child_list: Vec<T::KittyIndex> = Vec::new();
                child_list.push(child_id);
                kitty.children = Some(child_list);
            }

            if let Some(mut partner) = kitty.breed_partners {
                partner.push(partner_id);
                kitty.breed_partners = Some(partner);
            } else {
                let mut partner: Vec<T::KittyIndex> = Vec::new();
                partner.push(partner_id);
                kitty.breed_partners = Some(partner);
            }
        }
    }

    fn do_bread(
        sender: &T::AccountId,
        kitty_id1: T::KittyIndex,
        kitty_id2: T::KittyIndex,
    ) -> sp_std::result::Result<T::KittyIndex, DispatchError> {
        let kitty1 = Self::kitties(kitty_id1).ok_or(Error::<T>::InvalidKittyId)?;
        let kitty2 = Self::kitties(kitty_id2).ok_or(Error::<T>::InvalidKittyId)?;

        ensure!(kitty_id1 != kitty_id2, Error::<T>::RequireDifferentParent);

        let kitty_id = Self::next_kitty_id()?;

        let kitty1_dna = kitty1.dna;
        let kitty2_dna = kitty2.dna;
        let selector = Self::random_value(&sender);
        let mut new_dna = [0u8; 16];

        for i in 0..kitty1_dna.len() {
            new_dna[i] = Self::combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
        }

        let kitty = Kitty {
            kitty_id: kitty_id,
            dna: new_dna,
            owner: (*sender).clone(),
            parents: Some((kitty_id1, kitty_id2)),
            children: None,
            breed_partners: None,
        };
        Self::insert_kitty(sender, kitty_id, &kitty);

        // update kitty
        Self::update_kitty(kitty_id1, kitty_id, kitty_id2);
        Self::update_kitty(kitty_id2, kitty_id, kitty_id1);
        Ok(kitty_id)
    }

    fn get_parent(kitty_id: T::KittyIndex) -> Option<(T::KittyIndex, T::KittyIndex)> {
        if let Some(kitty) = Self::kitties(kitty_id) {
            return kitty.parents;
        }
        None
    }

    fn get_children(kitty_id: T::KittyIndex) -> Option<Vec<T::KittyIndex>> {
        if let Some(kitty) = Self::kitties(kitty_id) {
            return kitty.children;
        }
        None
    }

    // Q4
    fn get_brothers(kitty_id: T::KittyIndex) -> Option<Vec<T::KittyIndex>> {
        if let Some(kitty) = Self::kitties(kitty_id) {
            if let Some((p1, p2)) = kitty.parents {
                let mut brothers: Vec<_> = Vec::<T::KittyIndex>::new();
                if let Some(children1) = Self::get_children(p1) {
                    let mut chidlren1_mut = children1.to_vec();
                    brothers.append(&mut chidlren1_mut);
                }
                if let Some(children2) = Self::get_children(p2) {
                    let mut chidlren2_mut = children2.to_vec();
                    brothers.append(&mut chidlren2_mut);
                }
                let brothers_except_self: Vec<_> = brothers
                    .iter()
                    .filter(|k| *k != &kitty_id)
                    .map(|k| *k)
                    .collect();
                return Some(brothers_except_self);
            }
        }
        None
    }
}
