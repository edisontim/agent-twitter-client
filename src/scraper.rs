use crate::api::client::TwitterClient;
use crate::auth::user_auth::TwitterUserAuth;
use crate::constants::BEARER_TOKEN;
use crate::error::Result;
use crate::error::TwitterError;
use crate::messages::DirectMessagesResponse;
use crate::models::{Profile, Tweet};
use crate::search::{fetch_search_tweets, SearchMode};
use crate::timeline::v1::{QueryProfilesResponse, QueryTweetsResponse};
use crate::timeline::v2::QueryTweetsResponse as V2QueryTweetsResponse;
use serde_json::Value;

pub struct Scraper {
    pub twitter_client: TwitterClient,
}

impl Scraper {
    pub async fn new() -> Result<Self> {
        let auth = TwitterUserAuth::new(BEARER_TOKEN.to_string()).await?;
        let twitter_client = TwitterClient::new(auth.clone())?;
        Ok(Self { twitter_client })
    }

    pub async fn login(
        &mut self,
        username: String,
        password: String,
        email: Option<String>,
        two_factor_secret: Option<String>,
    ) -> Result<()> {
        if let Some(user_auth) = self
            .twitter_client
            .auth
            .as_any()
            .downcast_ref::<TwitterUserAuth>()
        {
            let mut auth = user_auth.clone();
            auth.login(
                &self.twitter_client.client,
                &username,
                &password,
                email.as_deref(),
                two_factor_secret.as_deref(),
            )
            .await?;

            self.twitter_client.auth = auth.clone();
            //self.client = TwitterClient::new(Box::new(auth))?;
            Ok(())
        } else {
            Err(TwitterError::Auth("Invalid auth type".into()))
        }
    }

    pub async fn get_profile(&self, username: &str) -> Result<crate::models::Profile> {
        crate::profile::get_profile(&self.twitter_client, username).await
    }
    pub async fn send_tweet(
        &self,
        text: &str,
        reply_to: Option<&str>,
        media_data: Option<Vec<(Vec<u8>, String)>>,
    ) -> Result<Value> {
        crate::tweets::create_tweet_request(&self.twitter_client, text, reply_to, media_data).await
    }

    pub async fn get_home_timeline(
        &self,
        count: i32,
        seen_tweet_ids: Vec<String>,
    ) -> Result<Vec<Value>> {
        crate::timeline::home::fetch_home_timeline(&self.twitter_client, count, seen_tweet_ids)
            .await
    }

    pub async fn save_cookies(&self, cookie_file: &str) -> Result<()> {
        if let Some(user_auth) = self
            .twitter_client
            .auth
            .as_any()
            .downcast_ref::<TwitterUserAuth>()
        {
            user_auth.save_cookies_to_file(cookie_file).await
        } else {
            Err(TwitterError::Auth("Invalid auth type".into()))
        }
    }

    pub async fn get_cookie_string(&self) -> Result<String> {
        if let Some(user_auth) = self
            .twitter_client
            .auth
            .as_any()
            .downcast_ref::<TwitterUserAuth>()
        {
            user_auth.get_cookie_string().await
        } else {
            Err(TwitterError::Auth("Invalid auth type".into()))
        }
    }

    pub async fn set_cookies(&mut self, json_str: &str) -> Result<()> {
        if let Some(user_auth) = self
            .twitter_client
            .auth
            .as_any()
            .downcast_ref::<TwitterUserAuth>()
        {
            let mut auth = user_auth.clone();
            auth.set_cookies(json_str).await?;

            self.twitter_client.auth = auth.clone();
            self.twitter_client = TwitterClient::new(auth)?;
            Ok(())
        } else {
            Err(TwitterError::Auth("Invalid auth type".into()))
        }
    }

    pub async fn set_from_cookie_string(&mut self, cookie_string: &str) -> Result<()> {
        if let Some(user_auth) = self
            .twitter_client
            .auth
            .as_any()
            .downcast_ref::<TwitterUserAuth>()
        {
            let mut auth = user_auth.clone();
            auth.set_from_cookie_string(cookie_string).await?;

            self.twitter_client.auth = auth.clone();
            self.twitter_client = TwitterClient::new(auth)?;
            Ok(())
        } else {
            Err(TwitterError::Auth("Invalid auth type".into()))
        }
    }

    pub async fn get_followers(
        &self,
        user_id: &str,
        count: i32,
        cursor: Option<String>,
    ) -> Result<(Vec<Profile>, Option<String>)> {
        crate::relationships::get_followers(&self.twitter_client, user_id, count, cursor).await
    }

    pub async fn get_following(
        &self,
        user_id: &str,
        count: i32,
        cursor: Option<String>,
    ) -> Result<(Vec<Profile>, Option<String>)> {
        crate::relationships::get_following(&self.twitter_client, user_id, count, cursor).await
    }

    pub async fn follow_user(&self, username: &str) -> Result<()> {
        crate::relationships::follow_user(&self.twitter_client, username).await
    }

    pub async fn unfollow_user(&self, username: &str) -> Result<()> {
        crate::relationships::unfollow_user(&self.twitter_client, username).await
    }

    pub async fn send_quote_tweet(
        &self,
        text: &str,
        quoted_tweet_id: &str,
        media_data: Option<Vec<(Vec<u8>, String)>>,
    ) -> Result<Value> {
        crate::tweets::create_quote_tweet(&self.twitter_client, text, quoted_tweet_id, media_data)
            .await
    }

    pub async fn fetch_tweets_and_replies(
        &self,
        username: &str,
        max_tweets: i32,
        cursor: Option<&str>,
    ) -> Result<V2QueryTweetsResponse> {
        crate::tweets::fetch_tweets_and_replies(&self.twitter_client, username, max_tweets, cursor)
            .await
    }
    pub async fn fetch_tweets_and_replies_by_user_id(
        &self,
        user_id: &str,
        max_tweets: i32,
        cursor: Option<&str>,
    ) -> Result<V2QueryTweetsResponse> {
        crate::tweets::fetch_tweets_and_replies_by_user_id(
            &self.twitter_client,
            user_id,
            max_tweets,
            cursor,
        )
        .await
    }
    pub async fn fetch_list_tweets(
        &self,
        list_id: &str,
        max_tweets: i32,
        cursor: Option<&str>,
    ) -> Result<Value> {
        crate::tweets::fetch_list_tweets(&self.twitter_client, list_id, max_tweets, cursor).await
    }

    pub async fn like_tweet(&self, tweet_id: &str) -> Result<Value> {
        crate::tweets::like_tweet(&self.twitter_client, tweet_id).await
    }

    pub async fn retweet(&self, tweet_id: &str) -> Result<Value> {
        crate::tweets::retweet(&self.twitter_client, tweet_id).await
    }

    pub async fn create_long_tweet(
        &self,
        text: &str,
        reply_to: Option<&str>,
        media_ids: Option<Vec<String>>,
    ) -> Result<Value> {
        crate::tweets::create_long_tweet(&self.twitter_client, text, reply_to, media_ids).await
    }

    pub async fn get_tweet(&self, id: &str) -> Result<(Option<Tweet>, Option<Vec<Tweet>>)> {
        crate::tweets::get_tweet(&self.twitter_client, id).await
    }

    pub async fn search_tweets(
        &self,
        query: &str,
        max_tweets: i32,
        search_mode: SearchMode,
        cursor: Option<String>,
    ) -> Result<QueryTweetsResponse> {
        fetch_search_tweets(&self.twitter_client, query, max_tweets, search_mode, cursor).await
    }

    pub async fn search_profiles(
        &self,
        query: &str,
        max_profiles: i32,
        cursor: Option<String>,
    ) -> Result<QueryProfilesResponse> {
        crate::search::search_profiles(&self.twitter_client, query, max_profiles, cursor).await
    }

    pub async fn get_user_tweets(
        &self,
        user_id: &str,
        count: i32,
        cursor: Option<String>,
    ) -> Result<V2QueryTweetsResponse> {
        crate::tweets::fetch_user_tweets(&self.twitter_client, user_id, count, cursor.as_deref())
            .await
    }

    pub async fn get_direct_message_conversations(
        &self,
        screen_name: &str,
        cursor: Option<&str>,
    ) -> Result<DirectMessagesResponse> {
        crate::messages::get_direct_message_conversations(&self.twitter_client, screen_name, cursor)
            .await
    }

    pub async fn send_direct_message(&self, conversation_id: &str, text: &str) -> Result<Value> {
        crate::messages::send_direct_message(&self.twitter_client, conversation_id, text).await
    }
}
